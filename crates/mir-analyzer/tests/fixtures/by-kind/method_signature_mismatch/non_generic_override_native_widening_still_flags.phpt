===description===
Negative control for the generic-override-native-hint fix: an ordinary,
non-generic override that widens its native return type beyond the direct
ancestor's own native hint must still be flagged — the fix is scoped to
comparisons that only fail because of class-level template substitution.
===config===
suppress=UnusedParam
===file===
<?php
class Base {}
class Concrete extends Base {}
class Other extends Base {}

abstract class Container {
    abstract public function get(): Concrete;
}

final class BadContainer extends Container {
    public function get(): Other {
        return new Other();
    }
}
===expect===
MethodSignatureMismatch@11:4-11:34: Method BadContainer::get() signature mismatch: return type 'Other' is not a subtype of parent 'Concrete'
