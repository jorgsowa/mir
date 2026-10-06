===description===
A type alias used inside a Closure()/callable() signature's return or
param position (e.g. `Closure(): IntList`) never expanded at all --
expand_aliases_in_atomic had no TCallable/TClosure arm, so the alias
name inside the signature stayed unexpanded and the closure's return
type stayed mixed.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @psalm-type IntList = array<int> */
class Repo {
    /** @param Closure(): IntList $factory */
    public function viaClosureReturn(Closure $factory): void {
        $list = $factory();
        foreach ($list as $v) {
            strlen($v);
//                 ^^ ArgumentTypeCoercion: Argument $string of strlen() expects 'string', got 'int' — coercion may fail at runtime
        }
    }
}
