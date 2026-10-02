===description===
A `@phpstan-assert`/`@psalm-assert` call whose argument is a property chain (`$this->dto->id`) must not report the synthetic `this->dto` narrowing key as an unused variable.
===config===
suppress=MissingConstructor,UnusedParam
===file===
<?php
abstract class TestCase {
    /**
     * @template T
     * @param T $expected
     * @phpstan-assert =T $actual
     */
    final public static function assertSame(mixed $expected, mixed $actual): void {}

    /**
     * @template T
     * @param T $expected
     * @psalm-assert =T $actual
     */
    final public static function assertSamePsalm(mixed $expected, mixed $actual): void {}
}

final class Dto { public function __construct(public int $id) {} }

final class SomeTest extends TestCase
{
    private Dto $dto;

    public function setUp(): void
    {
        $this->dto = new Dto(1);
    }

    public function testPhpstan(): void
    {
        self::assertSame(1, $this->dto->id);
        $dto = $this->dto;
        /** @mir-check $dto is Dto */
        echo $dto->id;
    }

    public function testPsalm(): void
    {
        self::assertSamePsalm(1, $this->dto->id);
    }

    public function testLocalChain(): void
    {
        $dto = new Dto(2);
        self::assertSame(2, $dto->id);
    }
}
===expect===
