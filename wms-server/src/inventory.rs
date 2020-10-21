#[derive(Clone, Debug)]
pub struct Inventory {
    pub wms_services: Vec<WmsInventoryEntry>,
}

#[derive(Clone, Debug)]
pub struct WmsInventoryEntry {
    /// WMS base path like `/wms/qgs/ne`
    pub wms_path: String,
}
