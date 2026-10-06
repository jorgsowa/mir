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
     */
    public function give(): void{
        /** @var mixed */
        $model = null;
        $class = get_class($model);
        $class::foo();
    }
}
===expect===
UnusedSuppress@4:17-4:30: Suppress annotation for 'MixedArgument' is never used
