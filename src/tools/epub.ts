import ePub from "epubjs";

export interface ParsedChapter {
  index: number;
  title: string;
  content: string;
  tts_generated?: boolean;
}

export interface EpubMetadata {
  rawTitle: string;
  author: string | null;
  publisher: string | null;
  description: string | null;
}

export interface EpubParseResult {
  fileName: string;
  metadata: EpubMetadata;
  chapters: ParsedChapter[];
}

/**
 * 解析 EPUB 文件，提取元数据和章节文本内容
 */
export async function parseEpub(
  data: ArrayBuffer,
  fileName: string,
): Promise<EpubParseResult> {
  const book = ePub(data);
  await book.ready;

  const meta = await book.loaded.metadata;

  // --- Chapters（从 navigation 目录提取章节结构和标题）---
  const nav = await book.loaded.navigation;
  const tocItems: { label: string; href: string }[] = flattenToc(nav.toc);
  const spineItems = (book.spine as any).spineItems;
  const archive = (book as any).archive;
  const chapters: ParsedChapter[] = [];

  for (let i = 0; i < tocItems.length; i++) {
    const { label, href } = tocItems[i];
    if (!href) {
      continue;
    }

    // 去掉 href 中的锚点（如 "chapter1.xhtml#p1" → "chapter1.xhtml"）
    const cleanHref = href.split("#")[0];
    const fileNamePart = cleanHref.split("/").pop() || cleanHref;
    const matchedSpine = spineItems.find((s: any) => s.href.includes(fileNamePart));

    // 优先使用 canonical（完整路径如 /OEBPS/Text/ss06.xhtml），其次尝试 base + href
    const textHref = matchedSpine
      ? (matchedSpine.canonical || (archive.base + matchedSpine.href).replace(/\/\//g, '/'))
      : cleanHref;

    let rawHtml: string | null = null;
    try {
      rawHtml = await archive.getText(textHref);
    } catch (_e) {
      /* ignore */
    }

    // 如果 textHref 拿不到，尝试用 matchedSpine.url（原始 getText 通常接受的格式）
    if (!rawHtml || typeof rawHtml !== 'string' || rawHtml.trim().length === 0) {
      if (matchedSpine?.url) {
        try {
          const altRaw = await archive.getText(matchedSpine.url);
          if (altRaw && typeof altRaw === 'string' && altRaw.trim().length > 0) {
            rawHtml = altRaw;
          }
        } catch (_e2) {
          /* ignore */
        }
      }
    }

    if (!rawHtml || typeof rawHtml !== 'string' || rawHtml.trim().length === 0) {
      continue;
    }

    // XHTML 文件可能以 <?xml ...?> 声明开头，剥离它，否则 DOMParser 在 text/html 模式下会解析失败
    let cleanHtml = rawHtml;
    if (cleanHtml.startsWith('<?xml')) {
      const xmlEnd = cleanHtml.indexOf('?>');
      if (xmlEnd !== -1) {
        cleanHtml = cleanHtml.substring(xmlEnd + 2).trim();
      }
    }

    let textContent = "";
    try {
      const parser = new DOMParser();
      const doc = parser.parseFromString(cleanHtml, 'text/html');
      const parseError = doc.querySelector('parsererror');
      if (parseError) {
        // 后备方案：用正则去掉标签
        textContent = cleanHtml.replace(/<[^>]*>/g, ' ').replace(/\s+/g, ' ').trim();
      } else if (doc?.body) {
        textContent = doc.body.textContent?.replace(/\s+/g, " ").trim() || "";
      }
    } catch (_e: any) {
      // 后备方案：用正则去掉标签
      textContent = cleanHtml.replace(/<[^>]*>/g, ' ').replace(/\s+/g, ' ').trim();
    }

    if (!textContent) {
      continue;
    }
    chapters.push({ index: i, title: label, content: textContent });
  }

  const fileNameWithoutExt = fileName.replace(/\.[^/.]+$/, "");

  return {
    fileName: fileNameWithoutExt,
    metadata: {
      rawTitle: meta.title || "",
      author: meta.creator || null,
      publisher: meta.publisher || null,
      description: meta.description || null,
    },
    chapters,
  };
}

/** 从 EPUB navigation 目录中展开所有条目（含嵌套子项） */
function flattenToc(items: any[]): { label: string; href: string }[] {
  const result: { label: string; href: string }[] = [];
  for (const item of items) {
    result.push({ label: item.label, href: item.href });
    if (item.subitems && item.subitems.length > 0) {
      result.push(...flattenToc(item.subitems));
    }
  }
  return result;
}
