import CryptoKit
import Foundation

public struct ContainerMetadata {
    private static let magic = "\0webc".data(using: .ascii)!

    public let sha256: String

    public init?(fileHandle: FileHandle) {
        do {
            try fileHandle.seek(toOffset: 0)
            guard 
                let magic = try fileHandle.read(upToCount: ContainerMetadata.magic.count),
                magic == ContainerMetadata.magic
            else { return nil }
            var sha = SHA256()
            sha.update(data: ContainerMetadata.magic)
//            var jsonData = Data()
//            var jsonReadingFinished = false
            while let data = try fileHandle.read(upToCount: 1024) {
                sha.update(data: data)
//                if !jsonReadingFinished {
//                    if let endIndex = data.firstIndex(of: 0) {
//                        jsonData.append(data.subdata(in: 0..<endIndex))
//                        jsonReadingFinished = true
//                    } else {
//                        jsonData.append(data)
//                    }
//                }
            }
            self.sha256 = sha.finalize().map { i64 in
                String(i64, radix: 16)
            }.joined()
        } catch {
            return nil
        }
    }

    public init?(url: URL) {
        guard let handle = try? FileHandle(forReadingFrom: url) else { return nil }
        self.init(fileHandle: handle)
    }
}
