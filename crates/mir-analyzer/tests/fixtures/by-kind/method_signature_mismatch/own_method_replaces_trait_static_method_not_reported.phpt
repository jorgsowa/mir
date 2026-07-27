===description===
FP: a class's own non-static method with the same name as a directly-used
trait's static method was flagged as a static/non-static override mismatch.
Same composition-time-precedence root cause as the final-method sibling in
final_method_overridden — a class member always wins over a same-named
trait method, no compatibility check at all (verified live: no fatal).
===file===
<?php
trait T {
    public static function greet(): string { return 'hi'; }
}
class Widget {
    use T;
    public function greet(): string { return 'bye'; }
}
===expect===
