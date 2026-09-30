===description===
Undefined callable class
===file===
<?php
class A {
    public function getFoo(): Foo
//                            ^^^ UndefinedClass: Class Foo does not exist
    {
        return new Foo([]);
//                 ^^^ UndefinedClass: Class Foo does not exist
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
