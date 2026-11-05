"use client";

import { FormEvent, useMemo, useState } from "react";

type Proposal = {
  id: string;
  daoName?: string;
  title: string;
  summary: string;
  author: string;
  category: string;
  status: "Active" | "Passed" | "Closed";
  ends: string;
  yes: number;
  no: number;
  abstain: number;
  quorum: number;
  voted?: "yes" | "no" | "abstain";
  discussion: string[];
};
const initial: Proposal[] = [
  {
    id: "SIP-024",
    title: "Fund the open documentation sprint",
    summary:
      "Allocate a small community grant to improve onboarding guides, API references, and contributor documentation for the next release cycle.",
    author: "Maya Chen",
    category: "Community",
    status: "Active",
    ends: "2 days left",
    yes: 184200,
    no: 42800,
    abstain: 12900,
    quorum: 300000,
    discussion: [
      "Clear docs help new contributors make a first useful change.",
      "Could we include accessibility review in the scope?",
    ],
  },
  {
    id: "SIP-023",
    title: "Set aside time for protocol research",
    summary:
      "Create a research working group to publish a short options paper on safer upgrade patterns and review expectations.",
    author: "Leo Park",
    category: "Protocol",
    status: "Active",
    ends: "5 days left",
    yes: 231500,
    no: 77200,
    abstain: 22300,
    quorum: 300000,
    discussion: ["A bounded research brief sounds helpful."],
  },
  {
    id: "SIP-022",
    title: "Publish a quarterly community recap",
    summary:
      "Prepare a plain-language summary of proposals, decisions, and learning notes from the last quarter.",
    author: "Amara Okafor",
    category: "Community",
    status: "Passed",
    ends: "Closed Oct 30",
    yes: 409800,
    no: 62200,
    abstain: 18900,
    quorum: 300000,
    discussion: [],
  },
  {
    id: "SIP-021",
    title: "Review treasury reporting format",
    summary:
      "Explore a simpler monthly reporting template with clear labels for balances, commitments, and assumptions.",
    author: "Noah Vale",
    category: "Treasury",
    status: "Closed",
    ends: "Closed Oct 22",
    yes: 106500,
    no: 128900,
    abstain: 31100,
    quorum: 300000,
    discussion: ["The group will revisit this with clearer examples."],
  },
];
const demoDaos = ["Open Source Guild", "Climate Commons", "Creative Protocol"];
initial.forEach((proposal) => (proposal.daoName = demoDaos[0]));
const addedProposalTitles = [
  [demoDaos[0], "Publish a contributor onboarding checklist", "Community"],
  [demoDaos[1], "Fund a neighborhood tree mapping pilot", "Grants"],
  [demoDaos[1], "Set a quarterly review for project outcomes", "Operations"],
  [demoDaos[1], "Create an open guide to native plant care", "Community"],
  [demoDaos[1], "Reserve funds for local data stewardship", "Treasury"],
  [demoDaos[1], "Invite two community reviewers for grant rounds", "Governance"],
  [demoDaos[2], "Sponsor an open source generative art toolkit", "Grants"],
  [demoDaos[2], "Publish a plain-language creator royalty guide", "Community"],
  [demoDaos[2], "Schedule a public demo and feedback session", "Operations"],
  [demoDaos[2], "Create a small fund for accessible event captions", "Treasury"],
  [demoDaos[2], "Document the proposal review process", "Governance"],
] as const;
const addedProposals: Proposal[] = addedProposalTitles.map(
  ([daoName, title, category], index): Proposal => ({
    id: `DEMO-${String(index + 1).padStart(3, "0")}`,
    daoName,
    title,
    summary: `Fictional ${category.toLowerCase()} proposal for ${daoName}. This local preview invites members to review the scope, discuss tradeoffs, and record a nonbinding sample vote.`,
    author: ["Riley Chen", "Sam Rivera", "Jamie Patel", "Alex Morgan"][index % 4],
    category,
    status: index % 3 === 0 ? "Active" : index % 3 === 1 ? "Passed" : "Closed",
    ends: index % 3 === 0 ? "6 days left" : "Closed Oct 18",
    yes: 82400 + index * 7820,
    no: 17400 + index * 1120,
    abstain: 4500 + index * 310,
    quorum: 300000,
    discussion: [],
  }),
);
const allSampleProposals = [...initial, ...addedProposals];

export default function SolGovPage() {
  const [proposals, setProposals] = useState(allSampleProposals);
  const [selectedDao, setSelectedDao] = useState("All demo DAOs");
  const [tab, setTab] = useState("All proposals");
  const [search, setSearch] = useState("");
  const [createOpen, setCreateOpen] = useState(false);
  const [title, setTitle] = useState("");
  const [summary, setSummary] = useState("");
  const [selectedVote, setSelectedVote] = useState<Record<string, "yes" | "no" | "abstain">>({});
  const [comment, setComment] = useState("");
  const [commenting, setCommenting] = useState<string | null>(null);
  const [notice, setNotice] = useState("");
  const visible = useMemo(
    () =>
      proposals.filter(
        (proposal) =>
          (selectedDao === "All demo DAOs" || proposal.daoName === selectedDao) &&
          (tab === "All proposals" || proposal.status === tab) &&
          `${proposal.id} ${proposal.title} ${proposal.summary} ${proposal.author} ${proposal.category}`
            .toLowerCase()
            .includes(search.toLowerCase()),
      ),
    [proposals, selectedDao, tab, search],
  );
  const castVote = (proposal: Proposal) => {
    const vote = selectedVote[proposal.id];
    if (!vote) {
      setNotice("Choose a sample vote option first.");
      return;
    }
    if (proposal.voted) {
      setNotice("This sample proposal already has a local vote.");
      return;
    }
    setProposals((current) =>
      current.map((item) =>
        item.id === proposal.id ? { ...item, [vote]: item[vote] + 1240, voted: vote } : item,
      ),
    );
    setNotice("Nonbinding vote recorded in local page state only.");
  };
  const createProposal = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const cleanTitle = title.trim();
    const cleanSummary = summary.trim();
    if (!cleanTitle || !cleanSummary) return;
    const proposal: Proposal = {
      id: `SIP-${String(25 + proposals.length - initial.length).padStart(3, "0")}`,
      title: cleanTitle.slice(0, 90),
      summary: cleanSummary.slice(0, 600),
      author: "You · local demo",
      category: "Community",
      status: "Active",
      ends: "7 days left",
      yes: 0,
      no: 0,
      abstain: 0,
      quorum: 300000,
      discussion: [],
    };
    setProposals((current) => [proposal, ...current]);
    setCreateOpen(false);
    setTitle("");
    setSummary("");
    setNotice("Sample proposal added locally. It is not published or on-chain.");
  };
  const addComment = (event: FormEvent<HTMLFormElement>, id: string) => {
    event.preventDefault();
    if (!comment.trim()) return;
    setProposals((current) =>
      current.map((item) =>
        item.id === id ? { ...item, discussion: [...item.discussion, comment.trim()] } : item,
      ),
    );
    setComment("");
    setNotice("Discussion note added to this local page.");
  };
  const formatVotes = (value: number) =>
    new Intl.NumberFormat("en", {
      notation: value > 999999 ? "compact" : "standard",
      maximumFractionDigits: 1,
    }).format(value);

  return (
    <main className="gov-app">
      <aside className="gov-sidebar">
        <a className="gov-brand" href="#overview">
          <span>◈</span> solgov <small>PREVIEW</small>
        </a>
        <button
          className="dao-switch"
          onClick={() => setNotice("DAO switching is not backed by a governance service.")}
        >
          <span className="dao-logo">S</span>
          <span>
            <b>Solstice Collective</b>
            <small>Community DAO · sample</small>
          </span>
          <i>⌄</i>
        </button>
        <nav>
          <button className="side-active" onClick={() => setTab("All proposals")}>
            ◫ <span>Overview</span>
          </button>
          <button onClick={() => setTab("Active")}>
            ◷ <span>Proposals</span>
            <b>2</b>
          </button>
          <button onClick={() => setNotice("Delegates are fictional samples in this workspace.")}>
            ♙ <span>Delegates</span>
          </button>
          <button
            onClick={() => setNotice("Treasury figures are illustrative; no wallet is connected.")}
          >
            ▤ <span>Treasury</span>
          </button>
          <button
            onClick={() =>
              setNotice("Governance forum is represented by local proposal discussion.")
            }
          >
            ▧ <span>Forum</span>
          </button>
        </nav>
        <div className="side-label">QUICK LINKS</div>
        <button
          className="quick-link"
          onClick={() => setNotice("Governance guidelines are not published in this concept.")}
        >
          ◎ Governance guide
        </button>
        <button
          className="quick-link"
          onClick={() => setNotice("Delegation is not connected to SPL Governance.")}
        >
          ⇢ Delegate voting power
        </button>
        <div className="gov-sidebar-bottom">
          <button onClick={() => setNotice("Settings are a visual concept only.")}>
            ⚙ Settings
          </button>
          <span>SAMPLE GOVERNANCE SPACE</span>
        </div>
      </aside>
      <section className="gov-main" id="overview">
        <header className="gov-topbar">
          <div className="gov-crumb">
            Solstice Collective <span>/</span> Overview{" "}
            <span className="sample-label">SAMPLE DAO</span>
          </div>
          <label className="gov-search">
            ⌕
            <input
              value={search}
              onChange={(event) => setSearch(event.target.value)}
              placeholder="Search proposals"
            />
          </label>
          <button
            className="gov-bell"
            onClick={() => setNotice("No live DAO alerts are connected.")}
          >
            ♧
          </button>
          <span className="gov-user">WI</span>
        </header>
        <div className="gov-content">
          <section className="gov-hero">
            <div>
              <span className="gov-eyebrow">COMMUNITY DECISIONS, MADE VISIBLE</span>
              <h1>
                Good governance
                <br />
                <em>is a shared practice.</em>
              </h1>
              <p>
                A sample space to review proposals, ask questions, and make collective decisions.
              </p>
            </div>
            <div className="gov-emblem" aria-hidden="true">
              <span>◈</span>
              <i />
              <i />
            </div>
          </section>
          <div className="data-banner">
            <b>ILLUSTRATIVE DAO</b>
            <span>
              Proposals, token balances, treasury figures, and participants are fictional sample
              data. Votes here have no governance effect.
            </span>
          </div>
          <section className="gov-stats">
            <article>
              <small>TREASURY VALUE · SAMPLE</small>
              <strong>$284,600</strong>
              <span>Illustrative only</span>
            </article>
            <article>
              <small>MEMBERS · SAMPLE</small>
              <strong>1,284</strong>
              <span>Community index</span>
            </article>
            <article>
              <small>ACTIVE PROPOSALS</small>
              <strong>02</strong>
              <span>In this sample space</span>
            </article>
            <article>
              <small>YOUR SAMPLE VOTING POWER</small>
              <strong>1,240 GOV</strong>
              <span>Not a wallet balance</span>
            </article>
          </section>
          <section className="proposal-section" aria-labelledby="dao-directory-title">
            <div className="proposal-heading">
              <div>
                <span className="gov-eyebrow">SAMPLE DIRECTORY</span>
                <h2 id="dao-directory-title">Explore the demo DAOs.</h2>
              </div>
              <button
                className={selectedDao === "All demo DAOs" ? "create-proposal" : "vote-selected"}
                aria-pressed={selectedDao === "All demo DAOs"}
                onClick={() => setSelectedDao("All demo DAOs")}
              >
                All three
              </button>
            </div>
            <div className="proposal-toolbar">
              <div>
                {demoDaos.map((dao) => (
                  <button
                    key={dao}
                    className={selectedDao === dao ? "filter-active" : ""}
                    aria-pressed={selectedDao === dao}
                    onClick={() => setSelectedDao(dao)}
                  >
                    {dao} · {proposals.filter((proposal) => proposal.daoName === dao).length}{" "}
                    proposals
                  </button>
                ))}
              </div>
              <span>Fictional groups · local preview</span>
            </div>
          </section>
          <section className="proposal-section">
            <div className="proposal-heading">
              <div>
                <span className="gov-eyebrow">GOVERNANCE / PROPOSALS</span>
                <h2>Make room for good questions.</h2>
              </div>
              <button className="create-proposal" onClick={() => setCreateOpen(true)}>
                ＋ New proposal
              </button>
            </div>
            <div className="proposal-toolbar">
              <div>
                {["All proposals", "Active", "Passed", "Closed"].map((item) => (
                  <button
                    key={item}
                    className={tab === item ? "filter-active" : ""}
                    onClick={() => setTab(item)}
                  >
                    {item}
                  </button>
                ))}
              </div>
              <span>{visible.length} sample proposals</span>
            </div>
            <div className="proposal-list">
              {visible.map((proposal) => {
                const total = proposal.yes + proposal.no + proposal.abstain;
                const yesWidth = total ? (proposal.yes / total) * 100 : 0;
                const noWidth = total ? (proposal.no / total) * 100 : 0;
                return (
                  <article className="proposal-card" key={proposal.id}>
                    <div className="proposal-id-row">
                      <span>
                        {proposal.daoName} <i>·</i> {proposal.id} <i>·</i> {proposal.category}
                      </span>
                      <b className={`status ${proposal.status.toLowerCase()}`}>{proposal.status}</b>
                    </div>
                    <h3>{proposal.title}</h3>
                    <p>{proposal.summary}</p>
                    <div className="proposal-author">
                      <span className="author-dot">
                        {proposal.author
                          .split(" ")
                          .slice(0, 2)
                          .map((part) => part[0])
                          .join("")}
                      </span>
                      <span>
                        Proposed by <b>{proposal.author}</b>
                      </span>
                      <small>{proposal.ends}</small>
                    </div>
                    {proposal.status === "Active" && (
                      <>
                        <div className="vote-bar">
                          <i style={{ width: `${yesWidth}%` }} />
                          <b style={{ width: `${noWidth}%` }} />
                          <em />
                        </div>
                        <div className="vote-summary">
                          <span>
                            <i className="yes-dot" /> For <b>{formatVotes(proposal.yes)}</b>
                          </span>
                          <span>
                            <i className="no-dot" /> Against <b>{formatVotes(proposal.no)}</b>
                          </span>
                          <span>
                            Abstain <b>{formatVotes(proposal.abstain)}</b>
                          </span>
                          <span>Quorum {formatVotes(proposal.quorum)}</span>
                        </div>
                        <div className="vote-actions">
                          <div role="group" aria-label={`Choose a vote for ${proposal.title}`}>
                            {(["yes", "no", "abstain"] as const).map((vote) => (
                              <button
                                key={vote}
                                className={
                                  selectedVote[proposal.id] === vote ? "vote-selected" : ""
                                }
                                onClick={() =>
                                  setSelectedVote((current) => ({
                                    ...current,
                                    [proposal.id]: vote,
                                  }))
                                }
                              >
                                {vote === "yes" ? "For" : vote === "no" ? "Against" : "Abstain"}
                              </button>
                            ))}
                          </div>
                          <button
                            className="cast-vote"
                            onClick={() => castVote(proposal)}
                            disabled={Boolean(proposal.voted)}
                          >
                            {proposal.voted ? "Vote recorded locally" : "Review vote"}
                          </button>
                        </div>
                      </>
                    )}
                    <div className="discussion-toggle">
                      <button
                        onClick={() =>
                          setCommenting(commenting === proposal.id ? null : proposal.id)
                        }
                      >
                        ▧ Discussion ({proposal.discussion.length})
                      </button>
                      <button
                        onClick={() =>
                          setNotice("Proposal sharing is not connected to a public DAO.")
                        }
                      >
                        ↗ Share
                      </button>
                    </div>
                    {commenting === proposal.id && (
                      <div className="discussion-panel">
                        {proposal.discussion.map((note, index) => (
                          <p key={`${proposal.id}-${index}`}>
                            <b>{index === 0 ? "Community member" : "You"}</b>
                            {note}
                          </p>
                        ))}
                        <form onSubmit={(event) => addComment(event, proposal.id)}>
                          <input
                            value={comment}
                            onChange={(event) => setComment(event.target.value)}
                            placeholder="Add a local discussion note"
                            aria-label="Discussion note"
                          />
                          <button disabled={!comment.trim()}>Post</button>
                        </form>
                      </div>
                    )}
                  </article>
                );
              })}
              {visible.length === 0 && (
                <div className="no-proposals">No sample proposals match this view.</div>
              )}
            </div>
          </section>
          <footer className="gov-footer">
            SOLGOV / LOCAL INTERFACE CONCEPT <span>Not connected to a DAO or Solana program</span>
          </footer>
        </div>
      </section>
      {createOpen && (
        <div
          className="gov-backdrop"
          onMouseDown={(event) => {
            if (event.target === event.currentTarget) setCreateOpen(false);
          }}
        >
          <form className="proposal-modal" onSubmit={createProposal}>
            <button
              type="button"
              className="modal-close"
              onClick={() => setCreateOpen(false)}
              aria-label="Close"
            >
              ×
            </button>
            <span className="gov-eyebrow">CREATE / LOCAL DRAFT</span>
            <h2>Bring a question to the table.</h2>
            <p>
              This draft is added only to the current browser session. It will not be submitted for
              an on-chain vote.
            </p>
            <label>
              Proposal title
              <input
                value={title}
                onChange={(event) => setTitle(event.target.value)}
                required
                maxLength={90}
                placeholder="What should the community consider?"
              />
            </label>
            <label>
              Summary
              <textarea
                value={summary}
                onChange={(event) => setSummary(event.target.value)}
                required
                rows={5}
                maxLength={600}
                placeholder="Share the context, options, and intended outcome."
              />
            </label>
            <button className="create-proposal" type="submit">
              Add local draft
            </button>
          </form>
        </div>
      )}
      {notice && (
        <div className="gov-toast" role="status">
          {notice}
          <button onClick={() => setNotice("")} aria-label="Dismiss">
            ×
          </button>
        </div>
      )}
    </main>
  );
}
