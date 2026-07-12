===description===
A final class named only in a method's `@return` docblock tag (no native
return type naming it) must not be reported UnusedClass.
===config===
suppress=
===file===
<?php
final class Widget {}

class Factory {
    /**
     * @return ?Widget
     */
    public function make(): mixed {
        return null;
    }
}

(new Factory())->make();
===expect===
