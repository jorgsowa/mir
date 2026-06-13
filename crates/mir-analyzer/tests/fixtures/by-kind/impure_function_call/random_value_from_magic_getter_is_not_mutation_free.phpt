===description===
Random value from magic getter is not mutation free
===file===
<?php
/**
 * @property int<1, 10> $b
 */
class A {
    /** @mutation-free */
    public function __get(string $key)
    {
        if ($key === "b") {
            return random_int(1, 10);
        }

        return null;
    }

    public function __set(string $key, string $value): void
    {
        throw new Exception("Setting not supported!");
    }
}

$a = new A;

/** @assert-if-true =1 $arg->b */
function assertBIsOne(A $arg): bool
{
    return $arg->b === 1;
}

if (assertBIsOne($a)) {
    takesOne($a->b);
}

/** @param 1 $_arg */
function takesOne(int $_arg): void {}

===expect===
MissingConstructor@5:0-5:9: Class A has uninitialized properties but no constructor
