===description===
inline suppression works at any nesting depth, not just top-level statements
===file===
<?php
class C {
    public function m(): void {
        if (true) {
            new NoSuchClass(); // @mir-ignore UndefinedClass
        }
    }
}
===expect===
