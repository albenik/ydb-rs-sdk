use crate::errors::*;
use crate::internal::discovery::{DiscoveryState, Service};
use http::Uri;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::{Arc, RwLock};

pub(crate) trait LoadBalancer {
    fn endpoint(&self, service: Service) -> Result<Uri>;
    fn set_discovery_state(&mut self, discovery_state: &Arc<DiscoveryState>) -> Result<()>;
}

#[derive(Clone)]
pub(crate) struct SharedLoadBalancer {
    inner: Arc<RwLock<Box<dyn LoadBalancer>>>,
}

impl SharedLoadBalancer {
    pub(crate) fn new(load_balancer: Box<dyn LoadBalancer>) -> Self {
        return Self {
            inner: Arc::new(RwLock::new(load_balancer)),
        };
    }
}

impl LoadBalancer for SharedLoadBalancer {
    fn endpoint(&self, service: Service) -> Result<Uri> {
        return self.inner.read()?.endpoint(service);
    }

    fn set_discovery_state(&mut self, discovery_state: &Arc<DiscoveryState>) -> Result<()> {
        self.inner.write()?.set_discovery_state(discovery_state)
    }
}

pub(crate) struct StaticLoadBalancer {
    endpoint: Uri,
}

impl StaticLoadBalancer {
    pub(crate) fn new(endpoint: Uri) -> Self {
        return Self { endpoint };
    }
}

impl LoadBalancer for StaticLoadBalancer {
    fn endpoint(&self, _: Service) -> Result<Uri> {
        return Ok(self.endpoint.clone());
    }

    fn set_discovery_state(&mut self, _: &Arc<DiscoveryState>) -> Result<()> {
        Err(Error::Custom(
            "static balancer no way to update state".into(),
        ))
    }
}

pub(crate) struct RoundRobin {
    counter: AtomicUsize,
    discovery_state: Arc<DiscoveryState>,
}

impl LoadBalancer for RoundRobin {
    fn endpoint(&self, service: Service) -> Result<Uri> {
        let counter = self.counter.fetch_add(1, Relaxed);
        let nodes = self.discovery_state.services.get(&service);
        match nodes {
            None => Err(Error::Custom(
                format!("no endpoints for service: '{}'", service).into(),
            )),
            Some(nodes) => {
                if nodes.len() > 0 {
                    let node = &nodes[counter % nodes.len()];
                    Ok(node.uri.clone())
                } else {
                    Err(Error::Custom(
                        format!("empty endpoint list for service: {}", service).into(),
                    ))
                }
            }
        }
    }

    fn set_discovery_state(&mut self, discovery_state: &Arc<DiscoveryState>) -> Result<()> {
        self.discovery_state = discovery_state.clone();
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::internal::discovery::Service::Table;
    use std::ops::Deref;
    use std::str::FromStr;

    struct MockBalancer {
        endpoint: Uri,
        endpoint_counter: AtomicUsize,
        set_updates_counter: AtomicUsize,
        last_state: Arc<DiscoveryState>,
    }

    impl MockBalancer {
        pub fn new(endpoint: Uri, state: DiscoveryState) -> Self {
            return Self {
                endpoint,
                endpoint_counter: Default::default(),
                set_updates_counter: Default::default(),
                last_state: Arc::new(state),
            };
        }
    }

    impl LoadBalancer for MockBalancer {
        fn endpoint(&self, service: Service) -> Result<Uri> {
            self.endpoint_counter.fetch_add(1, Relaxed);
            return Ok(self.endpoint.clone());
        }

        fn set_discovery_state(&mut self, discovery_state: &Arc<DiscoveryState>) -> Result<()> {
            self.last_state = discovery_state.clone();
            return UNIT_OK;
        }
    }

    #[test]
    fn shared_load_balancer() -> UnitResult {
        let test_uri = Uri::from_str("http://test.com")?;

        let s1 = SharedLoadBalancer::new(Box::new(MockBalancer::new(
            test_uri.clone(),
            DiscoveryState::default(),
        )));
        let s2 = s1.clone();

        assert_eq!(test_uri, s1.endpoint(Table)?);
        return UNIT_OK;
    }
}
