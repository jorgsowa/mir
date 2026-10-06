===description===
No crash on get class method call
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class User {
    /**
     * @suppress MixedArgument
//               ^^^^^^^^^^^^^ UnusedSuppress: Suppress annotation for 'MixedArgument' is never used
     */
    public function give(): void{
        /** @var mixed */
        $model = null;
        $class = get_class($model);
        $class::foo();
    }
}
===expect===
