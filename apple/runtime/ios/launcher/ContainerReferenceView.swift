import SwiftUI

struct ContainerReferenceView: View {
    let ref: ContainerReference

    var body: some View {
        VStack {
            Text("\(ref.metadata.name) v\(ref.metadata.version)")
            Text("Id: \(ref.metadata.id)")
            Text("SHA256: \(ref.metadata.sha256)")
                .lineLimit(1)
        }
        .navigationBarTitleDisplayMode(.inline)
        .navigationTitle("webrogue")
        .toolbar {
            Button("Run") {
                NotificationCenter.default.post(
                    name: .init(rawValue: "WebrogueRunPath"),
                    object: [ref.path, ref.dataPath]
                )
            }
        }
    }
}
