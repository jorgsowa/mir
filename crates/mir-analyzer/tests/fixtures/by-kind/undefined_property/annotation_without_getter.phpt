===description===
Annotation without getter
===file===
<?php
/**
 * @property bool $is_protected
 */
final class Page {
    public function isProtected(): bool
    {
        return $this->is_protected;
    }
}
===expect===
MissingConstructor@5:6-5:18: Class Page has uninitialized properties but no constructor
