===description===
Bound that is itself a template param is resolved before checking — no FP
===file===
<?php
interface IFoo {}
class Bar implements IFoo {}

class Binder {
    /**
     * @template A
     * @template B of A
     * @param interface-string<A> $interfaceName
     * @param class-string<B> $className
     */
    public function bind(string $interfaceName, string $className): void {
        $_ = [$interfaceName, $className];
    }
}

$binder = new Binder();
$binder->bind(IFoo::class, Bar::class);
===expect===
