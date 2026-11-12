===description===
`@return self::MAP[K]` on a method reads the class constant at the bound key.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
declare(strict_types=1);

namespace App;

final class Registry
{
    public const MAP = [
        'count' => 3,
        'label' => 'name',
    ];

    /**
     * @template K of key-of<self::MAP>
     * @param K $key
     * @return self::MAP[K]
     */
    public function get(string $key): int|string
    {
        return self::MAP[$key];
    }
}

function takesInt(int $value): void {}

function test(Registry $registry): void
{
    takesInt($registry->get('count'));
    takesInt($registry->get('label'));
    //       ^^^^^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $value of takesInt() expects 'int', got '"name"'
}
