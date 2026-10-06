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
//                  ^^^^^^^ UndefinedDocblockClass: Docblock type 'V' does not exist
}

$p = new IntProcessor();
$p->process("this should be an int, not a string");
//          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $v of process() expects 'int', got '"this should be an int, not a string"'
