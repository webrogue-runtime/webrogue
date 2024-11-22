import Combine
import Foundation

class ContainerReferenceViewModel: ObservableObject {
    @Published var decodedData = ""
    var terminate: (() -> Void)?
    @Published var isRunning = false

    private var data = Data()
    private let lock = NSLock()
    private var hasDecodingTask = false

    func append(_ data: Data) {
        self.data.append(data)
        planDecoding()
    }

    func clear() {
        lock.lock()
        data = Data()
        lock.unlock()
        planDecoding()
    }

    private func planDecoding() {
        guard !hasDecodingTask else { return }
        hasDecodingTask = true
        DispatchQueue.main.async {
            self.decode()
        }
    }

    private func decode() {
        assert(hasDecodingTask)
        lock.lock()
        defer { lock.unlock() }
        hasDecodingTask = false
        decodedData = String(decoding: data, as: UTF8.self)
    }
}
