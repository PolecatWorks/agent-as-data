use ::hams::hams::Hams;
use ::hams::probe::AsyncHealthProbe;
use ::hams::probe::FFIProbe;
use ::hams::probe::manual::Manual as ProbeManual;

pub struct HamsHarness {
    pub hams: Hams,
    pub ready_signal: ProbeManual,
}

impl HamsHarness {
    pub async fn init(mut hams: Hams) -> Result<Self, String> {
        let ready_signal = ProbeManual::new("db-connected", true);
        
        hams.ready_insert_async(Box::new(FFIProbe::from(ready_signal.clone())) as Box<dyn AsyncHealthProbe>).await;
        
        hams.start().map_err(|e| format!("Failed to start HaMS: {}", e))?;
        
        Ok(Self { hams, ready_signal })
    }
}
