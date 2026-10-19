===description===
A call into another package passes its arguments through unless it is a known
escaper (Laminas Escaper) or template renderer (Twig), or returns an object that
a persistence layer loads rather than builds from the argument.
===config===
<mir>
  <issueHandlers>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedArgument errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file:composer.json===
{
  "autoload": {
    "psr-4": {
      "Laminas\\Escaper\\": "vendor/laminas/laminas-escaper/src/",
      "Twig\\": "vendor/twig/twig/src/",
      "Doctrine\\ORM\\": "vendor/doctrine/orm/src/",
      "Acme\\": "vendor/acme/text/src/"
    }
  }
}
===file:vendor/laminas/laminas-escaper/src/Escaper.php===
<?php
namespace Laminas\Escaper;
class Escaper {
    public function escapeHtml(string $s): string { return $s; }
    public function escapeHtmlAttr(string $s): string { return $s; }
    public function escapeJs(string $s): string { return $s; }
    public function escapeCss(string $s): string { return $s; }
    public function escapeUrl(string $s): string { return $s; }
}
===file:vendor/twig/twig/src/Environment.php===
<?php
namespace Twig;
class Environment {
    public function render(string $name, array $context = []): string { return $name; }
    public function load(string $name): TemplateWrapper { return new TemplateWrapper(); }
}
===file:vendor/twig/twig/src/TemplateWrapper.php===
<?php
namespace Twig;
class TemplateWrapper {
    public function render(array $context = []): string { return ''; }
}
===file:vendor/doctrine/orm/src/EntityManager.php===
<?php
namespace Doctrine\ORM;
class EntityManager {
    /** @return object|null */
    public function find(string $class, mixed $id) { return null; }
}
===file:vendor/acme/text/src/Text.php===
<?php
namespace Acme;
class Text {
    public function render(string $s): string { return $s; }
}
===file:src/User.php===
<?php
namespace App;
class User {
    public function getName(): string { return 'n'; }
}
===file:src/MyEscaper.php===
<?php
namespace App;
class MyEscaper extends \Laminas\Escaper\Escaper {}
class Homemade {
    public function escapeHtml(string $s): string { return $s; }
}
===file:src/Page.php===
<?php
namespace App;
use Acme\Text;
use Doctrine\ORM\EntityManager;
use Laminas\Escaper\Escaper;
use Twig\Environment;
function escaped(Escaper $e): void {
    echo $e->escapeHtml((string) $_GET['q']);
    echo $e->escapeHtmlAttr((string) $_GET['q']);
    echo $e->escapeJs((string) $_GET['q']);
    echo $e->escapeCss((string) $_GET['q']);
    echo $e->escapeUrl((string) $_GET['q']);
    echo '<a href="' . $e->escapeHtmlAttr((string) $_GET['q']) . '">x</a>';
    shell_exec($e->escapeHtml((string) $_GET['q']));
}
function rendered(Environment $twig): void {
    echo $twig->render('x.twig', ['q' => (string) $_GET['q']]);
    echo $twig->load('x.twig')->render(['q' => (string) $_GET['q']]);
}
function loaded(EntityManager $em): void {
    $user = $em->find(User::class, (string) $_GET['id']);
    if ($user !== null) {
        echo $user->getName();
    }
    echo $em->find(User::class, (string) $_GET['id'])?->getName();
}
function passedThrough(Text $text): void {
    echo $text->render((string) $_GET['q']);
}
function subclassAndLookalike(MyEscaper $mine, Homemade $own): void {
    echo $mine->escapeHtml((string) $_GET['q']);
    echo $own->escapeHtml((string) $_GET['q']);
}
===expect===
Page.php: TaintedShell@14:4-14:51: Tainted shell command — possible command injection
Page.php: TaintedHtml@28:4-28:44: Tainted HTML output — possible XSS
Page.php: TaintedHtml@32:4-32:47: Tainted HTML output — possible XSS
