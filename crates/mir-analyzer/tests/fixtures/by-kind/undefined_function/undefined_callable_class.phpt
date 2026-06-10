===description===
Undefined callable class
===file===
<?php
class A {
    public function getFoo(): Foo
    {
        return new Foo([]);
    }

    /**
     * @param  mixed $argOne
     * @param  mixed $argTwo
     * @return void
     */
    public function bar($argOne, $argTwo)
    {
        $this->getFoo()($argOne, $argTwo);
    }
}
===expect===
UndefinedClass@3:31-3:34: Class Foo does not exist
UndefinedClass@5:20-5:23: Class Foo does not exist
