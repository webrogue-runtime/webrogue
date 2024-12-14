import SwiftUI
import WebrogueCommon

struct ContainerReferenceView: View {
    let ref: ContainerReference

    var body: some View {
        VStack {
            Text("Path: \(ref.path)")
            Text("SHA256: \(ref.metadata.sha256)")
                .lineLimit(1)
        }
            .navigationBarTitleDisplayMode(.inline)
            .navigationTitle("webrogue")
            .toolbar {
                Button("Run") {
                    WebrogueAppDelegate.shared?.runGame(path: ref.path) { _ in }
                }
            }
    }
}
