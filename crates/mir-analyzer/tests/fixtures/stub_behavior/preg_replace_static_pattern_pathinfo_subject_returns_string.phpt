===description===
`preg_replace()` with a static pattern and filename input returns a string.
===file===
<?php
final class Project {
    public function __construct(public string $slug) {}

    public static function fromPath(string $path): self {
        return new self(
            slug: preg_replace('/^\d+-/', '', pathinfo($path, PATHINFO_FILENAME)),
        );
    }
}
===expect===
