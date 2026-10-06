===description===
`@phpstan-assert =Type $x` (strict equality) narrows like `Type` for variables, properties and property chains, with a concrete or template type.
===config===
<mir>
  <issueHandlers>
    <MissingConstructor errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
abstract class TestCase {
    /**
     * @template T
     * @param T $expected
     * @phpstan-assert =T $actual
     */
    final public static function assertSame(mixed $expected, mixed $actual): void {}

    /** @phpstan-assert =int $actual */
    final public static function assertInt(mixed $actual): void {}
}

final class Dto { public function __construct(public int $id) {} }

final class SomeTest extends TestCase
{
    private Dto $dto;
    public mixed $raw;

    public function chained(): void
    {
        self::assertSame(1, $this->dto->id);
        $id = $this->dto->id;
        /** @mir-check $id is int */
        echo $id;
    }

    public function concreteOnProperty(): void
    {
        self::assertInt($this->raw);
        $raw = $this->raw;
        /** @mir-check $raw is int */
        echo $raw;
    }

    public function variable(mixed $value): void
    {
        self::assertInt($value);
        /** @mir-check $value is int */
        echo $value;
    }
}
