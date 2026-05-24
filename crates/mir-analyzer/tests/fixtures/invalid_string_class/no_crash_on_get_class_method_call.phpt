===description===
noCrashOnGetClassMethodCall
===file===
<?php
class User {
    /**
     * @psalm-suppress MixedArgument
     */
    public function give(): void{
        /** @var mixed */
        $model = null;
        $class = get_class($model);
        $class::foo();
    }
}
===expect===
InvalidStringClass
===ignore===
TODO
