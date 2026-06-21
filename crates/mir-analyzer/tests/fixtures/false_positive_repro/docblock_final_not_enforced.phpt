===description===
FP-J: @final is a soft docblock convention, not the PHP `final` keyword. Extending
a class annotated with @final (but not the keyword) must not emit InvalidExtendClass.
===file===
<?php

/** @final */
class Base {
    public function greet(): string { return 'hello'; }
}

// Extending a @final class is allowed — @final is only an IDE hint.
class Child extends Base {
    public function greet(): string { return 'hi'; }
}
===expect===
