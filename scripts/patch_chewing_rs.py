path = r'c:\Users\Defu\Documents\gis-code\chewing-kanata-lab\upstream\windows-chewing-tsf\tip\src\text_service\chewing.rs'
with open(path, 'r', encoding='utf-8') as f:
    content = f.read()

target1 = """        if !self.is_composing() {
            let shape_mode = self.chewing_editor.editor_options().character_form;"""

replacement1 = """        if !self.is_composing() {
            if evt.ksym.is_ascii() {
                let ascii_char = evt.ksym.to_unicode();
                let policy_ctx = PolicyContext::new(PolicyMode::AsciiFirst);
                if should_bypass_key(ascii_char, &policy_ctx) {
                    debug!(ascii_char; "ASCII-first policy: bypass key to application");
                    return Ok(false);
                }
            }
            let shape_mode = self.chewing_editor.editor_options().character_form;"""

target2 = """            let mut commit = self.chewing_editor.display_commit().to_owned();
            self.chewing_editor.ack();
            if let Some(param) = text_action {
                commit.push_str(&param);
            }
            commit"""

replacement2 = """            let mut commit = self.chewing_editor.display_commit().to_owned();
            self.chewing_editor.ack();
            if let Some(param) = text_action {
                commit.push_str(&param);
            }
            let policy_ctx = PolicyContext::new(PolicyMode::AsciiFirst);
            commit = transform_commit(&commit, &policy_ctx).to_owned();
            commit"""

# normalize line endings for matching
c_norm = content.replace('\r\n', '\n')
t1_norm = target1.replace('\r\n', '\n')
t2_norm = target2.replace('\r\n', '\n')

if t1_norm in c_norm:
    c_norm = c_norm.replace(t1_norm, replacement1.replace('\r\n', '\n'), 1)
    print("Replaced target1 successfully")
else:
    print("Failed to find target1")

if t2_norm in c_norm:
    c_norm = c_norm.replace(t2_norm, replacement2.replace('\r\n', '\n'), 1)
    print("Replaced target2 successfully")
else:
    print("Failed to find target2")

with open(path, 'w', encoding='utf-8', newline='\r\n') as f:
    f.write(c_norm)
