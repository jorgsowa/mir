===description===
Source data a method stores in one of its class's properties is tainted when
another method, a subclass or a typed caller reads that property. Escaping
before the write, or a write of clean data, isn't reported. A write through a
nested property or array element taints the outer property.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedOperand errorLevel="suppress"/>
    <MixedPropertyFetch errorLevel="suppress"/>
    <MixedPropertyAssignment errorLevel="suppress"/>
    <PropertyNotSetInConstructor errorLevel="suppress"/>
    <MissingPropertyType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Search {
    public string $query = '';
    private string $escaped = '';
    private string $label = 'Search';
    private static string $last = '';
    private array $filters = [];

    public function load(): void {
        $q = (string) $_GET['q'];
        $this->query = $q;
        $this->escaped = htmlspecialchars($q);
        self::$last = $q;
        $this->filters['tag'] = $_GET['tag'];
    }

    public function render(): void {
        echo $this->query;
//      ^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
        echo $this->escaped;
        echo $this->label;
        echo self::$last;
//      ^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
        echo $this->filters['tag'];
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }

    public function query(): string {
        return $this->query;
    }
}

class AdvancedSearch extends Search {
    public function renderAdvanced(): void {
        echo $this->query;
//      ^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}

function show(Search $search): void {
    echo $search->query;
//  ^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo $search->query();
//  ^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

class Settings {
    public ?Settings $child = null;
    public string $title = '';
}

function nested(Settings $settings): void {
    $settings->child = new Settings();
    $settings->child->title = (string) $_GET['title'];
    echo $settings->child->title;
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}
