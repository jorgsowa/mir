===description===
An `@inheritDoc` override inherits the parent's docblock param type that binds the inherited template, so the template resolves from the argument as it does through the parent.
===file===
<?php
interface Shape {}
interface Named {}
final class Circle implements Shape, Named {}
final class Square implements Shape, Named {}

interface Registry
{
    /**
     * @template T of Shape&Named
     * @param string|class-string<T> $name
     * @return Closure():T
     */
    public function get(string $name): Closure;
}

final class Impl implements Registry
{
    /** @inheritDoc */
    public function get(string $name): Closure
    {
        return fn() => new Circle();
    }
}

/** @param Closure():Circle $factory */
function useCircle(Closure $factory): void { $factory(); }

function viaParent(Registry $registry): void
{
    useCircle($registry->get(Circle::class));
}

function viaImplementation(Impl $impl): void
{
    useCircle($impl->get(Circle::class));
    $factory = $impl->get(Square::class);
    /** @mir-check $factory is Closure(): Square */
    $factory();
}

function plainStringStillAccepted(Impl $impl): void
{
    $impl->get('Circle');
}
