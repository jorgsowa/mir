===description===
Annotation without getter
===ignore===
TODO
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
