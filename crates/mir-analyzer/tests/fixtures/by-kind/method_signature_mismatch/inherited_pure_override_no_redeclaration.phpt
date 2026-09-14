===description===
An implementation inherits an interface method's @pure contract without
needing to repeat the annotation, so a valid pure override must not produce
a MethodSignatureMismatch.
===file===
<?php
interface Formatter {
    /** @pure */
    public function format(string $value): string;
}

class TrimFormatter implements Formatter {
    public function format(string $value): string {
        return $value;
    }
}
===expect===
