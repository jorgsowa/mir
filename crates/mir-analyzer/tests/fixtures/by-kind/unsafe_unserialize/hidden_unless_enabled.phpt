===description===
`UnsafeUnserialize` is opt-in: without an `<issueHandlers>` entry giving it a
level, nothing is reported.
===file===
<?php
function load(string $stored): mixed {
    return unserialize($stored);
}
