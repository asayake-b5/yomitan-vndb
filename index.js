import {
    Dictionary,
    DictionaryIndex,
    TermEntry,
    KanjiEntry,
} from 'yomichan-dict-builder';

import {
    cheerioJsonMapper
} from 'cheerio-json-mapper';
import {
    DownloaderHelper
} from 'node-downloader-helper';
import * as fs from 'node:fs';
import AdmZip from 'adm-zip';


(async () => {

  const vndb_id = process.argv[2];
  const zip_name = process.argv[3];
  const dict_name = process.argv[4];

    const dictionary = new Dictionary({
      fileName: zip_name,
    });

  const url = `https://vndb.org/${vndb_id}/chars?view=0S-D0gqwdpO#chars`;
    const html = await fetch(url)
        .then(function(response) {
            return response.text()
        })
        .then(function(html) {
            return html;
        })
        .catch(function(err) {
            console.log('Failed to fetch page: ', err);
        });

    const customPipes = {
        /** returns alias or nothing */
        parseAlias: ({
            value
        }) => {
            console.log(value?.toString());
            return value?.toString().includes("Aliases") ? value?.toString().replace("Aliases", "") : '';
        },

    };


    const template = {
        title: "body > main:nth-child(4) > article:nth-child(1) > h1:nth-child(2)",
        characters: [{
            "$": ".chardetails",
            fullname: "thead small",
            engname: "thead td a",
            gender: "abbr || attr:title",
            aliases: "tbody > tr:first-of-type > td | parseAlias",
            hair: ".trait_group_i1 > td:nth-child(2)",
            eyes: ".trait_group_i35 > td:nth-child(2)",
            body: ".trait_group_i36 > td:nth-child(2)",
            clothes: ".trait_group_i37 > td:nth-child(2)",
            items: ".trait_group_i38 > td:nth-child(2)",
            description: ".chardesc > p",
            img: "> .charimg img || attr:src"
        }]

    };

    const results = await cheerioJsonMapper(html, template, {
        pipeFns: customPipes
    });
    console.log(results);

    // index
    const index = new DictionaryIndex()
        .setTitle(dict_name)
        .setRevision(`1.0vndb-${dict_name}`)
        .setAuthor('Asayake')
        .setDescription(`VNDB Deck of names for ${results.title}`)
        .setAttribution('test')
        .setUrl('https://example.com')
        .build();

    await dictionary.setIndex(index);

    fs.rmSync("images", {
        recursive: true,
        force: true
    });
    fs.mkdirSync("images");
    let i = 0;


  const todo = results.characters.length;
    for (const result of results.characters) {
        const names = result.fullname.split(/[, ・]/);
        const aliases = result.aliases.split(/[, ・]/);
        const all_duped = names.concat(aliases).concat(result.fullname).concat(result.aliases).filter(function(e) {
            return e
        });
        const all = Array.from(new Set(all_duped));
      if (result.img) {
        const dl = new DownloaderHelper(result.img, "images", {
            fileName: `${i}.jpg`,
            retry: {
                maxRetries: 5,
                delay: 100
            }
        });
        dl.on('error', (err) => console.log('Download Failed', err));
        dl.start().catch(err => console.error(err));

      }

        let definition = `
  ${result.fullname} (${result.gender}) \n`;
        if (result.aliases) {
            definition += `Also known as ${result.aliases} \n`;
        }
        if (result.hair) {
            definition += `Hair: ${result.hair} \n`;
        }
        if (result.eyes) {
            definition += `Eyes: ${result.eyes} \n`;
        }
        if (result.body) {
            definition += `Body: ${result.body} \n`;
        }
        if (result.clothes) {
            definition += `Clothes: ${result.clothes} \n`;
        }
        if (result.items) {
            definition += `Items: ${result.items} \n`;
        }
        if (result.description) {
            definition += `${result.description} \n`;
        }

      console.log(all);
        for (const name of all) {
            let entry = new TermEntry(name)
                .setReading(result.engname)
                .addDetailedDefinition(name);
          if(result.img) {
            entry = entry.addDetailedDefinition({
                  type: 'structured-content',
                  content: {
                    tag: 'img',
                    path: `images/${i}.jpg`,
                    width: 50,
                    height: 50,
                    // title?: string,
                    // description?: string,
                  //   // pixelated?: boolean,
                  //   imageRendering?: 'auto' | 'pixelated' | 'crisp-edges',
                  //   appearance?: 'auto' | 'monochrome',
                  //   background?: boolean,
                    collapsed: false,
                    collapsible: false,
                  //   verticalAlign?:
                  //     | 'baseline'
                  //     | 'sub'
                  //     | 'super'
                  //     | 'text-top'
                  //     | 'text-bottom'
                  //     | 'middle'
                  //     | 'top'
                  //     | 'bottom',
                  //   sizeUnits?: 'px' | 'em',
                  // }
                  }})
          }
                entry = entry.addDetailedDefinition(definition)
                .build();
            await dictionary.addTerm(entry);
        }

      i += 1;
      console.log(`${i}/${todo}`)
      await new Promise(r => setTimeout(r, 500));
    };
        await new Promise(r => setTimeout(r, 1000));

    const stats = await dictionary.export('./');
  var zip = new AdmZip(`./${zip_name}`);
  zip.addLocalFolder("images", "images");
  zip.writeZip(`./${zip_name}`);
    console.log('Done exporting!');
    console.table(stats);
    fs.rmSync("images", {
        recursive: true,
        force: true
    });
})();
