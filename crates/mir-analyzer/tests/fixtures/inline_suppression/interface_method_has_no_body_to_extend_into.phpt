===description===
An abstract/interface method declaration ends in `;`, not a body — the
body-extension scan must recognize there is no `{ ... }` to extend into and
must NOT swallow the following, unrelated declaration's own diagnostic.
===file===
<?php
interface I {
    /** @psalm-suppress UndefinedClass */
    public function a(): void;

    public function b(): NoSuchClass;
//                       ^^^^^^^^^^^ UndefinedClass: Class NoSuchClass does not exist
}
===expect===
UnusedSuppress@3:24-3:38: Suppress annotation for 'UndefinedClass' is never used
