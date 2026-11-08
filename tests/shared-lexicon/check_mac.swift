// Copyright (c) 2026 AI Chewing contributors. MIT License.
// Headless contract probe: uses the actual Mac decoder, no installed IME or user data.
import Foundation

@main struct CrossPlatformProbe {
    static func main() throws {
        let data = try Data(contentsOf: URL(fileURLWithPath: CommandLine.arguments[1]))
        let document = try LexiconDocument.decode(data)
        guard document.entries.count == 7,
              Set(document.entries.map(\.entry_id)).count == 7,
              document.entries.filter({ $0.reading == "H2O" }).count == 2,
              document.entries.contains(where: { $0.reading == "FinFET" }),
              document.entries.contains(where: { $0.reading == "finfet" }) else {
            throw LexiconError.invalid("Cross-platform fixture identity or Unicode loss")
        }
        let restored = try LexiconDocument.decode(document.exchangeData())
        guard restored == document else { throw LexiconError.invalid("Round trip changed data") }
        print("PASS: 7 shared v1 entries; identity, exact Unicode/case, Mac decode/export/decode")
    }
}
