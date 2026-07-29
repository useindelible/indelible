import SwiftUI

struct SaveSheetView: View {
    let url: String
    let onSave: () -> Void
    let onCancel: () -> Void
    let onSignIn: () -> Void

    @State private var isSaving = false

    var body: some View {
        VStack(spacing: 0) {
            Spacer()
            VStack(alignment: .leading, spacing: 16) {
                HStack(spacing: 8) {
                    Image(systemName: "bookmark.fill")
                        .foregroundColor(.accentColor)
                    Text("Save to Indelible")
                        .font(.headline)
                }

                Divider()

                Text(url)
                    .font(.subheadline)
                    .foregroundColor(.secondary)
                    .lineLimit(2)

                Text("Inbox (default)")
                    .font(.caption)
                    .foregroundColor(.secondary)

                if isSaving {
                    HStack {
                        ProgressView()
                        Text("Saving…")
                            .font(.subheadline)
                            .foregroundColor(.secondary)
                    }
                } else {
                    HStack(spacing: 12) {
                        Button(action: onCancel) {
                            Text("Cancel")
                                .frame(maxWidth: .infinity)
                        }
                        .buttonStyle(.bordered)

                        Button(action: {
                            isSaving = true
                            onSave()
                        }) {
                            Text("Save")
                                .frame(maxWidth: .infinity)
                        }
                        .buttonStyle(.borderedProminent)
                    }
                }
            }
            .padding(24)
            .background(Color(UIColor.systemBackground))
            .cornerRadius(16)
            .padding(.horizontal, 16)
            .padding(.bottom, 8)
        }
        .background(Color.black.opacity(0.3))
        .ignoresSafeArea()
    }
}
