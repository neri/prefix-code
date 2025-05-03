/// Playground for Compression
///
/// Copyright (c) 2025 Nerry, ALL RIGHTS RESERVED
///
/// https://github.com/neri/prefix-code
///
const APP_NAME = "Playground for Compression";
const COPYRIGHT = "Copyright (c) 2025 Nerry, ALL RIGHTS RESERVED.";
const REPOSITORY_URL = "https://github.com/neri/prefix-code";
const VERSION_STRING = "0.1.0";

import './style.css';
import * as libprefix from '../lib/libprefix';
import { HASH } from "./hash";
import { Dialog } from './dialog';

const $ = (x: string) => document.querySelector(x);

const alert = (x: string) => new AlertDialog(x).show();

const debounce = (fn: () => void, interval: number) => {
    let timer: NodeJS.Timeout | null = null;
    return () => {
        if (timer != null) {
            clearTimeout(timer);
        }
        timer = setTimeout(() => fn(), interval)
    }
}

class App {

    private currentTitle = '';
    private baseName = 'download';

    triggerCompress = debounce(() => this.updateResult(), 500);

    onload() {
        const html = $('html') as HTMLElement;

        document.title = APP_NAME;
        document.querySelectorAll('.app_name').forEach((element, _key, _parent) => {
            const appNameTextNode = document.createTextNode(APP_NAME);
            element.parentElement?.replaceChild(appNameTextNode, element);
        });
        document.querySelectorAll('.copyright').forEach((element, _key, _parent) => {
            const copyrightTextNode = document.createTextNode(COPYRIGHT);
            element.parentElement?.replaceChild(copyrightTextNode, element);
        });
        document.querySelectorAll('.repository_button').forEach((element, _key, _parent) => {
            const repositoryButton = document.createElement('a');
            repositoryButton.className = 'button';
            repositoryButton.href = REPOSITORY_URL;
            repositoryButton.target = "_blank";
            repositoryButton.appendChild(app.makeIcon('ic_launch'));
            repositoryButton.appendChild(document.createTextNode(' Open in GitHub'));
            element.parentElement?.replaceChild(repositoryButton, element);
        });
        document.querySelectorAll('.app_version').forEach((element, _key, _parent) => {
            const appVerTextNode = document.createTextNode(`Version: ${VERSION_STRING} Hash: ${HASH}`);
            element.parentElement?.replaceChild(appVerTextNode, element);
        });

        ($('#inputText') as HTMLTextAreaElement | null)?.addEventListener('keyup', (e: Event) => {
            this.triggerCompress();
        });

        ($('#execButton') as HTMLButtonElement | null)?.addEventListener('click', () => {
            this.updateResult();
        });

        ($('#maxBitSize') as HTMLSelectElement | null)?.addEventListener('change', () => {
            this.updateResult();
        });

        ($('body') as HTMLBodyElement).style.display = 'block';

        // new MainMenu().show();
    }

    private dimCount = 0;
    dim(x = 1) {
        const tag = $('#main') as HTMLDivElement | null;
        if (tag === null) { return; }

        this.dimCount += x;

        if (this.dimCount > 0) {
            tag.classList.add('blur');
        } else {
            tag.classList.remove('blur');
        }
    }

    makeIcon(icon_id: string): HTMLElement {
        const iconElement = document.createElement('span');
        iconElement.className = icon_id;
        iconElement.appendChild(document.createTextNode("\u00a0"));
        return iconElement;
    }

    updateResult() {
        const inputText = ($('#inputText') as HTMLTextAreaElement | null)?.value || "";
        this.doCompress(inputText);
    }

    doCompress(inputText: string) {
        const resultArea = ($('#resultArea') as HTMLElement);
        resultArea.innerText = "";
        const maxBitSize = parseInt(($('#maxBitSize') as HTMLSelectElement | null)?.value || "") || 0;
        if (inputText.length === 0) {
            // ($('#encodedText') as HTMLTextAreaElement).value = "";
            return
        };

        const input = new TextEncoder().encode(inputText);
        let result: string;
        try {
            result = libprefix.encode_chc(input, maxBitSize);
        } catch (e) {
            {
                const hrTag = document.createElement('hr');
                resultArea.appendChild(hrTag);

                const h2tag = document.createElement('h2');
                h2tag.className = 'error';
                h2tag.appendChild(document.createTextNode(`ERROR: ${e}`));
                resultArea.appendChild(h2tag);
            }
            return;
        }
        const json = JSON.parse(result) as EncodeChcResult;
        // console.log("Result: ", json);

        const encodedBase64 = arrayToHex(json.encoded_zlib, ' ');
        //arrayBufferToBase64(json.encoded_zlib);
        // ($('#encodedText') as HTMLTextAreaElement).value = encodedBase64;

        {
            const hrTag = document.createElement('hr');
            resultArea.appendChild(hrTag);

            const h2tag = document.createElement('h2');
            h2tag.appendChild(document.createTextNode("RESULT:"));
            resultArea.appendChild(h2tag);
        }

        {
            const h3Tag = document.createElement('h3');
            h3Tag.appendChild(document.createTextNode("Input:"));
            resultArea.appendChild(h3Tag);

            const detailsTag = document.createElement('details');
            detailsTag.open = false;

            const summaryTag = document.createElement('summary');
            const entropy = json.input_entropy.toLocaleString('en', { minimumFractionDigits: 3, maximumFractionDigits: 3 });
            const message = `# length ${json.input_len}, entropy ${entropy}`;
            summaryTag.appendChild(document.createTextNode(message));
            detailsTag.appendChild(summaryTag);

            const preTag = document.createElement('div');
            preTag.className = 'code';
            preTag.appendChild(document.createTextNode(arrayToHex(input)));
            detailsTag.appendChild(preTag);

            resultArea.appendChild(detailsTag);
        }

        {
            const h3tag = document.createElement('h3');
            h3tag.appendChild(document.createTextNode("Encoded Codes:"));
            resultArea.appendChild(h3tag);

            const detailsTag = document.createElement('details');
            detailsTag.open = true;

            const summaryTag = document.createElement('summary');
            const rate = (json.output_bits / json.input_bits * 100.0).toLocaleString('en', { minimumFractionDigits: 3, maximumFractionDigits: 3 });
            const ideal_bits = Math.ceil(json.input_len * json.input_entropy);
            const ideal_rate = (json.output_bits / ideal_bits * 100.0).toLocaleString('en', { minimumFractionDigits: 3, maximumFractionDigits: 3 });
            let message = `# ${json.output_bits} bits <= ${json.input_bits} bits (${rate}%)`;
            if (ideal_bits > 0) {
                message += `, ideal ${ideal_bits} bits (${ideal_rate}%)`;
            }
            summaryTag.appendChild(document.createTextNode(message));
            detailsTag.appendChild(summaryTag);

            const preTag = document.createElement('div');
            preTag.className = 'code';
            preTag.appendChild(document.createTextNode(json.encoded_str.join(' ')));
            detailsTag.appendChild(preTag);

            resultArea.appendChild(detailsTag);
        }

        {
            const h3tag = document.createElement('h3');
            h3tag.appendChild(document.createTextNode("Output like zlib:"));
            resultArea.appendChild(h3tag);

            const detailsTag = document.createElement('details');
            detailsTag.open = false;

            const summaryTag = document.createElement('summary');
            const actualRate = (json.encoded_zlib.length / json.input_len * 100.0).toLocaleString('en', { minimumFractionDigits: 3, maximumFractionDigits: 3 });
            const entropy = json.output_entropy.toLocaleString('en', { minimumFractionDigits: 3, maximumFractionDigits: 3 });
            const message = `# length ${json.encoded_zlib.length} (${actualRate}%), entropy ${entropy}`;
            summaryTag.appendChild(document.createTextNode(message));
            detailsTag.appendChild(summaryTag);

            const preTag = document.createElement('div');
            preTag.className = 'code';
            preTag.appendChild(document.createTextNode(encodedBase64));
            detailsTag.appendChild(preTag);

            resultArea.appendChild(detailsTag);
        }

        {
            const encodedWebpBase64 = arrayToHex(json.encoded_webp, ' ');
            //arrayBufferToBase64(json.encoded_webp);

            const h3tag = document.createElement('h3');
            h3tag.appendChild(document.createTextNode("Output like webp:"));
            resultArea.appendChild(h3tag);

            const detailsTag = document.createElement('details');
            detailsTag.open = false;

            const summaryTag = document.createElement('summary');
            const actualRate = (json.encoded_webp.length / json.input_len * 100.0).toLocaleString('en', { minimumFractionDigits: 3, maximumFractionDigits: 3 });
            const entropy = json.webp_entropy.toLocaleString('en', { minimumFractionDigits: 3, maximumFractionDigits: 3 });
            const message = `# length ${json.encoded_webp.length} (${actualRate}%), entropy ${entropy}`;
            summaryTag.appendChild(document.createTextNode(message));
            detailsTag.appendChild(summaryTag);

            const preTag = document.createElement('div');
            preTag.className = 'code';
            preTag.appendChild(document.createTextNode(encodedWebpBase64));
            detailsTag.appendChild(preTag);

            resultArea.appendChild(detailsTag);
        }

        {
            try {
                const decoded =
                    libprefix.decode_chc(json.encoded_zlib, input.length);

                if (decoded.toString() === input.toString()) {
                    // ok, identical
                } else {
                    const divTag = document.createElement('div');
                    divTag.className = 'error';
                    divTag.appendChild(document.createTextNode(`DECODE FAILED, PLEASE REPORT DETAILS`));
                    resultArea.appendChild(divTag);
                }
            } catch (e) {
                const divTag = document.createElement('div');
                divTag.className = 'error';
                divTag.appendChild(document.createTextNode(`DECODE ERROR: ${String(e)}, PLEASE REPORT DETAILS`));
                resultArea.appendChild(divTag);
            }
        }

        {
            const hrTag = document.createElement('hr');
            resultArea.appendChild(hrTag);

            const h2tag = document.createElement('h2');
            h2tag.appendChild(document.createTextNode("Prefix Table:"));
            resultArea.appendChild(h2tag);

            const tableTag = document.createElement('table');
            {
                const trTag = document.createElement('tr');
                const th1Tag = document.createElement('th');
                th1Tag.colSpan = 2;
                th1Tag.appendChild(document.createTextNode("Symbol"));
                trTag.appendChild(th1Tag);
                const th3Tag = document.createElement('th');
                th3Tag.colSpan = 2;
                th3Tag.appendChild(document.createTextNode("Frequency"));
                trTag.appendChild(th3Tag);
                const th5Tag = document.createElement('th');
                th5Tag.appendChild(document.createTextNode("Bit Length"));
                trTag.appendChild(th5Tag);
                const th6Tag = document.createElement('th');
                th6Tag.appendChild(document.createTextNode("Canonical Prefix Code"));
                trTag.appendChild(th6Tag);
                tableTag.appendChild(trTag);
            }

            json.prefix_table.forEach((item) => {
                const trTag = document.createElement('tr');

                const thTag = document.createElement('th');
                thTag.appendChild(document.createTextNode(String(item.symbol)));
                trTag.appendChild(thTag);

                const td1Tag = document.createElement('td');
                td1Tag.appendChild(document.createTextNode(String(item.symbol_char)));
                trTag.appendChild(td1Tag);

                const td2Tag = document.createElement('td');
                td2Tag.className = "right";
                td2Tag.appendChild(document.createTextNode(String(item.freq)));
                trTag.appendChild(td2Tag);

                const freqRate = item.freq_rate.toLocaleString('en', { minimumFractionDigits: 3, maximumFractionDigits: 3 });
                const td3Tag = document.createElement('td');
                td3Tag.appendChild(document.createTextNode(freqRate));
                trTag.appendChild(td3Tag);

                const td4Tag = document.createElement('td');
                td4Tag.className = "center";
                td4Tag.appendChild(document.createTextNode(String(item.len)));
                trTag.appendChild(td4Tag);

                const td5Tag = document.createElement('td');
                td5Tag.className = "right";
                td5Tag.appendChild(document.createTextNode(String(item.code)));
                trTag.appendChild(td5Tag);

                tableTag.appendChild(trTag);
            });
            resultArea.appendChild(tableTag);
        }

        if (json.huffman_tree.length > 0) {
            const h3tag = document.createElement('h3');
            h3tag.appendChild(document.createTextNode("Reference Huffman Tree:"));
            resultArea.appendChild(h3tag);

            const preTag = document.createElement('pre');
            // preTag.className = 'code';
            preTag.appendChild(document.createTextNode(json.huffman_tree));
            resultArea.appendChild(preTag);
        }

    }
}

const app = new App();

class AlertDialog extends Dialog {
    constructor(
        message: string,
        title: string | null = null,
        isModal: boolean = true,
        closeButton: boolean = true
    ) {
        super('dialogAlert', title ?? "", isModal, closeButton);

        const alertMessage = ($('#alertMessage') as HTMLElement | null);
        if (alertMessage !== null) {
            alertMessage.innerText = message;
        }
    }

    static dismiss() {
        new AlertDialog("").dismiss()
    }
}

class MainMenu extends Dialog {
    constructor() {
        super('dialogMainMenu', "", false);
    }
    onResetStyle(): void {
        this.frameElement.style.transform = 'translate(-100%, 0)';
    }
    onShow(): void {
        setTimeout(() => {
            this.frameElement.style.transform = 'translate(0, 0)';
        }, 50);
    }
}

class AboutDialog extends Dialog {
    constructor() {
        super('dialogAbout', "", false);
    }
}

class EncodeChcResult {
    input_len: number = 0;
    input_bits: number = 0;
    input_entropy: number = 0;
    output_bits: number = 0;
    output_entropy: number = 0;
    prefix_table: PrefixTableEntry[] = [];
    encoded_str: string[] = [];
    encoded_zlib: Uint8Array = new Uint8Array(0);
    encoded_webp: Uint8Array = new Uint8Array(0);
    webp_entropy: number = 0;
    huffman_tree: string = "";
}

class PrefixTableEntry {
    symbol: number = 0;
    symbol_char: string = "";
    freq: number = 0;
    freq_rate: number = 0;
    len: number = 0;
    code: string = "";
}


if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', app.onload);
} else {
    app.onload();
}

const arrayBufferToBase64 = (bytes: Uint8Array) => {
    let array = [];
    const len = bytes.length;
    for (let i = 0; i < len; i++) {
        array.push(String.fromCharCode(bytes[i]));
    }
    return btoa(array.join(''));
}

const arrayToHex = (bytes: Uint8Array, delimiter: string = ' ') => {
    const encoded: string[] = [];
    bytes.forEach((val) => encoded.push(val.toString(16).padStart(2, '0')));
    return encoded.join(delimiter);
}


const roundToEven = (v: number): number => {
    if (!Number.isFinite(v)) {
        return NaN;
    }
    const ipart = Math.floor(v);
    const fpart = v - ipart;
    if (fpart < 0.5) {
        return ipart;
    } else if (fpart > 0.5) {
        return Math.ceil(v);
    } else {
        if ((ipart & 1) == 0) {
            return ipart;
        } else {
            return Math.ceil(v);
        }
    }
}
