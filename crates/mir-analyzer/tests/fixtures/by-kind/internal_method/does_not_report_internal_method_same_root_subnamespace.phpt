===description===
Does not report internal method called from a sub-namespace under the same root namespace
===file:Console.php===
<?php
namespace Symfony\Component\Console;

class Output {
    /**
     * @internal
     */
    public function doWrite(): void {
    }
}
===file:Helper.php===
<?php
namespace Symfony\Component\Console\Helper;
$out = new \Symfony\Component\Console\Output();
$out->doWrite();
===expect===
