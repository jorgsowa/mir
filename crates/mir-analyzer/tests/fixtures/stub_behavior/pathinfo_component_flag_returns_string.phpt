===description===
`pathinfo()` with a literal component flag returns a string.
===file===
<?php
final class FileName {
    public function __construct(public string $value) {}
}

function filename(string $path): FileName {
    return new FileName(
        value: pathinfo($path, PATHINFO_FILENAME),
    );
}
===expect===
