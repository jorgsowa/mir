===description===
No crash on get class method call
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
UnusedPsalmSuppress@6:0-6:0: Suppress annotation for 'MixedArgument' is never used
