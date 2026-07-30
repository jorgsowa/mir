===description===
Negative control for the generic-override-native-hint fix: when the child
DOES restate its own (wrong) docblock refinement, the mismatch must still be
flagged — the fix only excuses a child that never opted into a narrower
promise in the first place.
===config===
suppress=UnusedParam
===file===
<?php
class Base {}
class Concrete extends Base {}
class Other extends Base {}

/** @template T of Base */
abstract class Container {
    /** @return T */
    abstract public function get(): Base;
}

/** @extends Container<Concrete> */
final class ConcreteContainer extends Container {
    /** @return Other */
    public function get(): Base {
        return new Other();
    }
}
===expect===
MethodSignatureMismatch@15:4-15:33: Method ConcreteContainer::get() signature mismatch: return type 'Other' is not a subtype of parent 'Concrete'
