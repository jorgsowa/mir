===description===
`@psalm-internal Vendor\Library` limits calls to that namespace and the ones nested in it, on a method or on the whole class. A call from a sibling namespace under the same root is `InternalMethod`.
===file:Library.php===
<?php
namespace Vendor\Library;

/**
 * @psalm-internal Vendor\Library
 */
final class Secret
{
    public static function token(): string
    {
        return 'secret';
    }

    public function reveal(): string
    {
        return 'secret';
    }
}

final class Helper
{
    /**
     * @psalm-internal Vendor\Library
     */
    public function assist(): void
    {
    }
}
===file:Nested.php===
<?php
namespace Vendor\Library\Nested;

use Vendor\Library\Helper;
use Vendor\Library\Secret;

function inside(Secret $secret, Helper $helper): string
{
    $helper->assist();
    return Secret::token() . $secret->reveal();
}
===file:App.php===
<?php
namespace Vendor\App;

use Vendor\Library\Helper;
use Vendor\Library\Secret;

function outside(Secret $secret, Helper $helper): string
{
    $helper->assist();
//  ^^^^^^^^^^^^^^^^^ InternalMethod: Method Vendor\Library\Helper::assist() is marked @internal
    return Secret::token() . $secret->reveal();
//         ^^^^^^^^^^^^^^^ InternalMethod: Method Vendor\Library\Secret::token() is marked @internal
//                           ^^^^^^^^^^^^^^^^^ InternalMethod: Method Vendor\Library\Secret::reveal() is marked @internal
}
