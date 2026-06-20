===description===
Does not report internal static method called from a sub-namespace under the same root namespace
===file:Console.php===
<?php
namespace Symfony\Component\Console;

class Output {
    /**
     * @internal
     */
    public static function create(): void {
    }
}
===file:Helper.php===
<?php
namespace Symfony\Component\Console\Helper;
\Symfony\Component\Console\Output::create();
===expect===
