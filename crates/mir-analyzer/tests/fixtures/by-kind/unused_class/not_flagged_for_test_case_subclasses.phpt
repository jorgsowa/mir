===description===
Test runners load `*TestCase` subclasses themselves
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
</mir>
===file===
<?php
namespace PHPUnit\Framework;

abstract class TestCase {}

namespace App\Tests;

use PHPUnit\Framework\TestCase;

abstract class KernelTestCase extends TestCase {}

final class UserTest extends TestCase {
    public function testName(): void {}
}

final class ControllerTest extends KernelTestCase {
    public function testIndex(): void {}
}
