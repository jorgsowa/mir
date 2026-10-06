===description===
Possibly null function call
===config===
<mir>
  <issueHandlers>
    <MixedReturnStatement errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @var Closure|null $foo
 */
$foo = null;


$foo =
    /**
     * @param mixed $bar
     * @suppress MixedFunctionCall
//               ^^^^^^^^^^^^^^^^^ UnusedSuppress: Suppress annotation for 'MixedFunctionCall' is never used
     */
    function ($bar) use (&$foo): string
    {
        if (is_array($bar)) {
            return $foo($bar);
        }

        return $bar;
    };
