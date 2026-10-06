===description===
int-mask-of<self::*> (no name prefix before the wildcard) matches every
literal-int constant on the class, not just ones sharing a common prefix.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Flags {
    const A = 1;
    const B = 2;

    /**
     * @param int-mask-of<self::*> $flags
     */
    public function set(int $flags): void {}
}

$f = new Flags();
$f->set(3);
$f->set(4);
//      ^ InvalidArgument: Argument $flags of set() expects '0|1|2|3', got '4'
