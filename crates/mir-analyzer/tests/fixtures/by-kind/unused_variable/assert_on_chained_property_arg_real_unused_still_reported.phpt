===description===
Skipping synthetic property-chain keys does not hide a genuinely unused local in the same method.
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
}

final class Dto { public function __construct(public int $id) {} }

final class SomeTest extends TestCase
{
    private Dto $dto;

    public function testRead(): void
    {
        $unused = 5;
//      ^^^^^^^ UnusedVariable: Variable $unused is never read
        self::assertSame(1, $this->dto->id);
    }
}
===expect===
