import type { Metadata } from "next";
import "./styles.css";

export const metadata: Metadata = {
  title: "SolGov — Governance Interface Showcase",
  description: "A local DAO governance concept with fictional proposals and nonbinding votes.",
  robots: { index: false, follow: false },
};

export default function RootLayout({ children }: Readonly<{ children: React.ReactNode }>) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  );
}
