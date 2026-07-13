===description===
A return type that DROPS a conjunct (fewer parts = broader, not a
covariant subtype) must still be flagged — the new intersection subtype
arm isn't a blanket "any intersection matches any intersection".
===file===
<?php
interface Boxed {
    public function getBox(): Countable&ArrayAccess&Iterator;
}
class Impl implements Boxed {
    public function getBox(): Countable&ArrayAccess {
        throw new RuntimeException();
    }
}
===expect===
MethodSignatureMismatch@6:4-6:53: Method Impl::getbox() signature mismatch: return type 'Countable&ArrayAccess' is not a subtype of parent 'Countable&ArrayAccess&Iterator'
