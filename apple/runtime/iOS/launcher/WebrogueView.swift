import SwiftUI
import UniformTypeIdentifiers

struct WebrogueView: View {
    @StateObject var containerStorage = ContainerStorage()
    @State var isFileImporterPresented = false

    var body: some View {
        NavigationView {
            Group {
                List(containerStorage.refs, id: \.metadata.sha256) { ref in
                    NavigationLink {
                        ContainerReferenceView(ref: ref)
                    } label: {
                        Text(ref.metadata.sha256)
                    }
                }
                Text("hi")

            }
                .navigationBarTitleDisplayMode(.inline)
                .navigationTitle("webrogue")
                .toolbar {
                    Button("Add") {
                        isFileImporterPresented = true
                    }
                }
                .fileImporter(
                    isPresented: $isFileImporterPresented,
                    allowedContentTypes: [.webc],
                    allowsMultipleSelection: false
                ) { result in
                    switch result {
                    case .success(let files):
                        for file in files {
                            let gotAccess = file.startAccessingSecurityScopedResource()
                            if !gotAccess { continue }
                            containerStorage.store(file)
                            file.stopAccessingSecurityScopedResource()
                        }
                    case .failure(_):
                        break
                    }
                }
        }
        .onReceive(NotificationCenter.default.publisher(for: .init(rawValue: "WebrogueURL"))) {
            guard let url = $0.object as? URL else { return }
            containerStorage.store(url)
        }
    }
}

#Preview {
    WebrogueView()
}
