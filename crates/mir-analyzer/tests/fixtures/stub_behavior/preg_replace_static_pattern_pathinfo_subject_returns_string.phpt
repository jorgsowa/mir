===description===
preg_replace with a valid static pattern and pathinfo's filename component returns
a string, so it can initialize a string constructor property without an array
alternative leaking from either call.
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
