use std::str;
use git2;
use pulldown_cmark::{ Parser, html, Event, Tag };
use maud::{ PreEscaped };
use maud_pulldown_cmark::Markdown;
use super::fa::{ FA, FAM };
use { RepositoryTreeEntry, RepositoryExtension };

fn find_readme(repo: &git2::Repository) -> Option<String> {
  let head_id = expect!(try_expect!(try_expect!(repo.head()).resolve()).target());
  let head = try_expect!(repo.find_commit(head_id));
  let tree = try_expect!(head.tree());
  let entry = expect!(tree.get_name("README").or_else(|| tree.get_name("README.md")));
  let object = try_expect!(entry.to_object(repo));
  let blob = expect!(object.as_blob());
  str::from_utf8(blob.content()).ok().map(|s| s.to_owned())
}

fn description(repo: &git2::Repository) -> Option<String> {
  // Render the readme and grab the first <p> element from it.
  find_readme(repo)
    .map(|readme| {
      let mut s = String::new();
      html::push_html(
        &mut s,
        Parser::new(&*readme)
          .skip_while(|ev| match *ev {
            Event::Start(Tag::Paragraph) => false,
            _ => true,
          })
          .take_while(|ev| match *ev {
            Event::End(Tag::Paragraph) => false,
            _ => true,
          }));
      s
    })
}

renderers! {
  Repository(repo: &'a git2::Repository) {
    @if let Some(readme) = find_readme(repo) {
      ^Markdown::from_string(&*readme)
    }
  }

  RepositoryStub(path: &'a str, name: &'a str, repo: &'a git2::Repository) {
    li.repo-stub {
      @match repo.origin_url() {
        Some(_) => ^FAM::Li(FA::CodeFork),
        None => ^FAM::Li(FA::Home),
      }
      a href={ ^path "/" ^name } {
        ^name
      }
      @if let Some(origin) = repo.origin_url() {
        " "
        small {
          "(fork of " ^super::MaybeLink(&origin, &origin) ")"
        }
      }
      @if let Some(description) = description(repo) {
        blockquote {
          ^PreEscaped(description)
        }
      }
    }
  }

  Repositories(path: &'a str, repos: &'a Vec<RepositoryTreeEntry>) {
    h1 { "Repositories" }
    ^RepositoriesList(path, repos)
  }

  RepositoriesList(path: &'a str, repos: &'a Vec<RepositoryTreeEntry>) {
    ul.fa-ul {
      @for entry in repos {
        @match *entry {
          RepositoryTreeEntry::Dir(ref name, ref repos) => {
            li {
              ^FAM::Li(FA::Sitemap)
              ^name
              ^RepositoriesList(&*(path.to_owned() + "/" + name), repos)
            }
          },
          RepositoryTreeEntry::Alias(ref alias, ref actual) => {
            li {
              ^FAM::Li(FA::Tag)
              a href={ ^path "/" ^alias } {
                ^alias
              }
              " alias of "
              a href=^actual {
                ^actual
              }
            }
          },
          RepositoryTreeEntry::Repo(ref name, ref repo) => {
            ^RepositoryStub(path, name, repo)
          },
        }
      }
    }
  }
}

impl<'a> super::repository_wrapper::RepositoryTab for &'a Repository<'a> {
  fn tab() -> super::repository_wrapper::Tab { super::repository_wrapper::Tab::Overview }
}
