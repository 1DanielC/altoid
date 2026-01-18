pub struct Camera{
    pub name: String,
    pub fs_strategy: FileStrategy,
    pub resource: String
}

enum FileStrategy {
    MOUNT,
    MTP
}