import SwiftUI
import WebrogueCommon
import UniformTypeIdentifiers

struct WebrogueView: View {
    @ObservedObject var wrappStorage = WebrogueAppDelegate.wrappStorage
    @State var isFileImporterPresented = false

    var body: some View {
        NavigationView {
            Group {
                List(wrappStorage.refs, id: \.metadata.sha256) { ref in
                    NavigationLink {
                        WrappRefView(ref: ref)
                    } label: {
                        Text(ref.metadata.sha256)
                    }
                }

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
                    allowedContentTypes: [.wrappType],
                    allowsMultipleSelection: false
                ) { result in
                    switch result {
                    case .success(let files):
                        for file in files {
                            let gotAccess = file.startAccessingSecurityScopedResource()
                            if !gotAccess { continue }
                            wrappStorage.store(file)
                            file.stopAccessingSecurityScopedResource()
                        }
                    case .failure(_):
                        break
                    }
                }
        }
    }
}

#Preview {
    WebrogueView()
}
