===description===
@implements Iface<Concrete> binds the interface's template param at call sites
===file===
<?php
/** @template V */
interface Processor {
    /** @param V $v */
    public function process($v): void;
}

/** @implements Processor<int> */
class IntProcessor implements Processor {
    /** @param V $v */
    public function process($v): void {}
}

$p = new IntProcessor();
$p->process("this should be an int, not a string");
===expect===
UnusedParam@11:28-11:30: Parameter $v is never used
InvalidArgument@15:12-15:49: Argument $v of process() expects 'int', got '"this should be an int, not a string"'
