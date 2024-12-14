public struct ContainerReference {
    public let path: String
    public let metadata: ContainerMetadata

    public init(path: String, metadata: ContainerMetadata) {
        self.path = path
        self.metadata = metadata
    }
}
