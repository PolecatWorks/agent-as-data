import { TestBed } from '@angular/core/testing';
import { HttpClientTestingModule, HttpTestingController } from '@angular/common/http/testing';
import { ApiService } from './api.service';

describe('ApiService Configurable Prefixes', () => {
  let service: ApiService;
  let httpMock: HttpTestingController;

  beforeEach(() => {
    TestBed.configureTestingModule({
      imports: [HttpClientTestingModule],
      providers: [ApiService]
    });
    service = TestBed.inject(ApiService);
    httpMock = TestBed.inject(HttpTestingController);
  });

  afterEach(() => {
    httpMock.verify();
  });

  it('should initialize with default baseUrl /api/aad/v1', () => {
    expect(service.getBaseUrl()).toBe('/api/aad/v1');
  });

  it('should direct HTTP requests to the default baseUrl', () => {
    service.getAgents().subscribe();
    const req = httpMock.expectOne('/api/aad/v1/agents/search');
    expect(req.request.method).toBe('POST');
    req.flush([]);
  });

  it('should normalize trailing slashes when setBaseUrl is called', () => {
    service.setBaseUrl('/api/custom/v1/');
    expect(service.getBaseUrl()).toBe('/api/custom/v1');

    service.getAgents().subscribe();
    const req = httpMock.expectOne('/api/custom/v1/agents/search');
    expect(req.request.method).toBe('POST');
    req.flush([]);
  });

  it('should load runtime config via loadAppConfig', async () => {
    const loadPromise = service.loadAppConfig('assets/contents/app-config.json');

    const configReq = httpMock.expectOne('assets/contents/app-config.json');
    expect(configReq.request.method).toBe('GET');
    configReq.flush({ apiPath: '/api/remote-runtime/v1', logLevel: 'INFO' });

    await loadPromise;
    expect(service.getBaseUrl()).toBe('/api/remote-runtime/v1');

    service.getAgents().subscribe();
    const agentsReq = httpMock.expectOne('/api/remote-runtime/v1/agents/search');
    agentsReq.flush([]);
  });

  it('NEGATIVE: rejects null, undefined, or empty prefix updates by preserving existing baseUrl', () => {
    service.setBaseUrl('/api/aad/v1');
    service.setBaseUrl('');
    expect(service.getBaseUrl()).toBe('/api/aad/v1');

    service.setBaseUrl(null as any);
    expect(service.getBaseUrl()).toBe('/api/aad/v1');

    service.setBaseUrl(undefined as any);
    expect(service.getBaseUrl()).toBe('/api/aad/v1');
  });

  it('NEGATIVE: loadAppConfig handles missing or empty apiPath gracefully by preserving default baseUrl', async () => {
    const loadPromise = service.loadAppConfig('assets/contents/empty-config.json');
    const req = httpMock.expectOne('assets/contents/empty-config.json');
    req.flush({});

    await loadPromise;
    expect(service.getBaseUrl()).toBe('/api/aad/v1');
  });

  it('NEGATIVE: loadAppConfig handles 404 or network errors gracefully by preserving default baseUrl', async () => {
    const loadPromise = service.loadAppConfig('assets/contents/not-found.json');
    const req = httpMock.expectOne('assets/contents/not-found.json');
    req.error(new ProgressEvent('error'), { status: 404, statusText: 'Not Found' });

    await loadPromise;
    expect(service.getBaseUrl()).toBe('/api/aad/v1');
  });
});
