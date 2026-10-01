<?php
namespace Psalm\Internal\Analyzer;

use Psalm\Codebase;
use Psalm\Config;
use Psalm\Internal\Provider\Providers;

class ProjectAnalyzer
{
    /** @var array<string, string> */
    private array $project_files = [];
    private Codebase $codebase;

    public function __construct(public Config $config, public Providers $providers)
    {
        $this->codebase = new Codebase($config);
    }

    public function getCodebase(): Codebase
    {
        return $this->codebase;
    }

    public function canReportIssues(string $file_path): bool
    {
        return isset($this->project_files[$file_path]);
    }
}
